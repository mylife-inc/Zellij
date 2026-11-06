Local sample media embedded by `zel new docs --full`. No third-party assets or
network downloads are used. The SVG was authored for the starter. The MP4 is a
1,691-byte, 32×32 gray sample copied from `scripts/fixtures/template.mp4`, generated
with:

```sh
ffmpeg -f lavfi -i color=c=gray:s=32x32:d=0.2 -c:v libx264 -pix_fmt yuv420p -movflags +faststart section-example.mp4
```

The video is intentionally tiny; replace it with product footage before publishing.
