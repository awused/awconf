package awconf

import (
	"errors"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

/**
Priority:
appname.toml
<executable directory>/appname.toml
$HOME/.appname.toml
$XDG_CONFIG_HOME/appname/appname.toml
/usr/local/etc/appname.toml
/usr/etc/appname.toml
*/

func LoadConfig(name string, conf interface{}) error {
	nametoml := name + ".toml"
	paths := []string{nametoml}

	ex, err := os.Executable()
	if err == nil {
		ex, err = filepath.EvalSymlinks(ex)
	}
	if err == nil {
		paths = append(paths, filepath.Join(filepath.Dir(ex), nametoml))
	}

	home := os.Getenv("HOME")
	if home != "" {
		paths = append(paths, filepath.Join(home, "."+nametoml))
	}

	xdg := os.Getenv("XDG_CONFIG_HOME")
	if xdg != "" {
		paths = append(paths, filepath.Join(xdg, name, nametoml))
	}

	paths = append(paths, filepath.Join("/usr/local/etc", nametoml))
	paths = append(paths, filepath.Join("/usr/etc", nametoml))

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
