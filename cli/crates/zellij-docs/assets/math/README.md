# Portable math assets

These files are embedded in `zel` and emitted only when a portable site contains
enabled math. Builds and readers never download them. `math.css` is Zellij's
font/layout integration; the remaining files are upstream snapshots. The OFL
text has one trailing space removed; its terms and wording are unchanged.

STIX Two Math 2.13b171 (SIL OFL 1.1) comes from the official STIX release commit
`744a22a4dd626cd14d75728aef34fc8ad7c85db0`:

- [Font](https://github.com/stipub/stixfonts/blob/744a22a4dd626cd14d75728aef34fc8ad7c85db0/fonts/static_otf_woff2/STIXTwoMath-Regular.woff2)
- [License](https://github.com/stipub/stixfonts/blob/744a22a4dd626cd14d75728aef34fc8ad7c85db0/OFL.txt)

The renderer fixes and MIT license come from `math-core` 0.9.0's source commit
`fc95cb704a8159467b69f1ebc93460de7c195ce1`:

- [CSS](https://github.com/tmke8/math-core/blob/fc95cb704a8159467b69f1ebc93460de7c195ce1/css/mathmlfixes.css)
- [License](https://github.com/tmke8/math-core/blob/fc95cb704a8159467b69f1ebc93460de7c195ce1/LICENSE)

SHA-256:

```text
094191335def3f0452c81ec0713cfc2f29bb6af8cecbf79b60881fbf2db97562  STIXTwoMath-Regular.woff2
242a676ce08515fc58af901e110589759cb426ac66ec917d37ce37d5dbf17945  OFL.txt
15128a36f105da7de7b382e5d7fd26dc1f9bc814a29d13f639d4681ff8eda679  mathmlfixes.css
4522db391501c9fde3553b377a9cec8bdd05eb80dd0bb3df51d9e44c3e4c8f2f  math-core-LICENSE
```

The browser gate checks the exported font's hash, actual local loading and
MathML layout. The font is unchanged, including its internal family names.
The CSS family alias keeps it separate from the site's body and heading fonts.
