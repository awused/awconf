package awconf

import (
	"errors"
	"flag"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

/**
Override config detection with the -awconf flag, but flags need to be parsed
by your executable to enable this. Specify /dev/null to use the empty config.

Priority:
./appname.toml
$HOME/.appname.toml
$XDG_CONFIG_HOME/appname/appname.toml
$XDG_CONFIG_HOME/appname/config.toml
/usr/local/etc/appname.toml
/usr/etc/appname.toml
<executable directory>/appname.toml
$GOBIN/appname.toml
$GOPATH/bin/appname.toml
*/

var override = flag.String("awconf", "", "Specify a config file to use")

// LoadConfig will attempt to find the config and load it into the provided reference.
func LoadConfig(name string, conf interface{}) error {
	if flag.Parsed() && *override != "" {
		_, err := toml.DecodeFile(*override, conf)
		return err
	}

	nametoml := name + ".toml"
	paths := []string{nametoml}

	home := os.Getenv("HOME")
	if home != "" {
		paths = append(paths, filepath.Join(home, "."+nametoml))
	}

	xdg := os.Getenv("XDG_CONFIG_HOME")
	if xdg != "" {
		paths = append(paths, filepath.Join(xdg, name, nametoml))
		paths = append(paths, filepath.Join(xdg, name, "config.toml"))
	} else {
		paths = append(paths, filepath.Join(home, ".config", name, nametoml))
		paths = append(paths, filepath.Join(home, ".config", name, "config.toml"))
	}

	paths = append(paths, filepath.Join("/usr/local/etc", nametoml))
	paths = append(paths, filepath.Join("/usr/etc", nametoml))

	ex, err := os.Executable()
	if err == nil {
		ex, err = filepath.EvalSymlinks(ex)
	}
	if err == nil {
		paths = append(paths, filepath.Join(filepath.Dir(ex), nametoml))
	}

	gobin := os.Getenv("GOBIN")
	if gobin != "" {
		paths = append(paths, filepath.Join(gobin, nametoml))
	}

	gopath := os.Getenv("GOPATH")
	if gopath != "" {
		paths = append(paths, filepath.Join(gopath, "bin", nametoml))
	}

	for _, p := range paths {
		if f, err := os.Stat(p); err == nil {
			if !f.IsDir() {
				_, err := toml.DecodeFile(p, conf)
				return err
			}
		}
	}

	return errors.New("Unable to find config file for " + name)
}
