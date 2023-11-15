#  Audio Plugins
Plugins using the [nih_plug framework](https://github.com/robbert-vdh/nih-plug)

# Create new plugin
Using nix
```shell
nix-shell -p python311 python311Packages.pipx
pipx run cookiecutter https://github.com/robbert-vdh/nih-plug-template.git
exit
```

## Building

```shell
cd <audio_plugin>
cargo xtask bundle <audio_plugin> --release
```
