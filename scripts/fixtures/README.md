`template.mp4` is a generated 32×32 gray video, used only to render section
scaffolds in CI. It contains no third-party media. Regenerate with:

```sh
ffmpeg -f lavfi -i color=c=gray:s=32x32:d=0.2 -c:v libx264 -pix_fmt yuv420p -movflags +faststart template.mp4
```
