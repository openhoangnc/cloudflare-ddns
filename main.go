package main

import (
	"bytes"
	"flag"
	"fmt"
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/hugomd/cloudflare-ddns/lib/providers"
	_ "github.com/hugomd/cloudflare-ddns/lib/providers/_all"
)

var client = http.Client{Timeout: 2 * time.Second}

func isNetError(err error) bool {
	_, ok := err.(net.Error)
	return ok
}

func httpGet(url string) (body string, err error) {
	fmt.Print(time.Now().Format("2006/01/02 15:04:05") + " Get " + url + " ")

	var rsp *http.Response
	for {
		fmt.Print(".")
		rsp, err = client.Get(url)
		if err != nil {
			if os.IsTimeout(err) {
				continue
			}
			if isNetError(err) {
				time.Sleep(client.Timeout)
				continue
			}
			return "", err
		}
		fmt.Println()
		break
	}
	defer rsp.Body.Close()

	buf, err := ioutil.ReadAll(rsp.Body)
	if err != nil {
		return "", err
	}

	return string(bytes.TrimSpace(buf)), nil
}

func checkIP() (string, error) {
	return httpGet("https://checkip.amazonaws.com")
}

func checkIP6() (string, error) {
	// has to be a service without A records (for ex api6.ipify.org cant be used)
	return httpGet("https://v6.ident.me")
}

func setEnvVarsFromConfig(filename *string) error {
	contents, err := ioutil.ReadFile(*filename)
	if err != nil {
		return err
	}

	lines := strings.Split(string(contents), "\n")
	for _, line := range lines {
		if strings.Contains(line, "=") {
			values := strings.SplitN(line, "=", 2)
			os.Setenv(values[0], values[1])
		}
	}

	return nil
}

func main() {
	var runonce bool
	var ticker *time.Ticker

	CheckDuration := flag.Duration("duration", 0, "update interval (ex. 15s, 1m, 6h); if not specified or set to 0s, run only once and exit")
	ConfigFile := flag.String("config", "", "location of an (optional) config file to load environment variables from")
	run4 := flag.Bool("ipv4", true, "update A record")
	run6 := flag.Bool("ipv6", false, "update AAAA record")
	flag.Parse()

	if *CheckDuration == time.Duration(0) {
		runonce = true
	} else {
		ticker = time.NewTicker(*CheckDuration)
	}

	if *ConfigFile != "" {
		err := setEnvVarsFromConfig(ConfigFile)
		if err != nil {
			panic(err)
		}
	}

	runDDNS(*run4, *run6)

	if runonce {
		os.Exit(0)
	}

	for range ticker.C {
		runDDNS(*run4, *run6)
	}
}

var lastIP = ""
var lastIP6 = ""

func runDDNS(run4, run6 bool) {
	PROVIDER := os.Getenv("PROVIDER")
	if PROVIDER == "" {
		log.Fatal("PROVIDER env. variable is required")
	}

	provider, err := providers.Providers[PROVIDER]()
	if err != nil {
		panic(err)
	}

	if run4 {
		ip, err := checkIP()
		if err != nil {
			panic(err)
		}

		log.Printf("IP is %s changed=%v", ip, ip != lastIP)
		if ip != lastIP {
			lastIP = ip
			err = provider.UpdateRecord(ip)
			if err != nil {
				panic(err)
			}
		}
	}

	if run6 {
		ip, err := checkIP6()
		if err != nil {
			panic(err)
		}

		log.Printf("IPv6 is %s changed=%v", ip, ip != lastIP6)
		if ip != lastIP6 {
			lastIP6 = ip
			err = provider.UpdateRecord6(ip)
			if err != nil {
				panic(err)
			}
		}
	}
}
