# ggbar

Status bar for Herbstluftwm.

```
ggbar x-offset y-offset width height monitor-num
```
E.g. `ggbar 0 0 1920 20 0`

## FAQ

- Why? Polybar doesn't really support HerbstluftWM. Also making this
was easier than figuring out how to configure Polybar.
- Why Rust? Good question! As it turns out, its a bit pointless if you just
use bindings for XCB and Cairo.
- How to configure? Edit the source code.

