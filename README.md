# Rusty K-Compressor

Port of [kcomp](https://github.com/mawngo/kcomp) project to Rust for learning purposes.

Reduce the number of colors used in the image using k-mean clustering.

### Options

```
> kcomprs -h  
Reduce number of colors used in image

Usage: kcomprs [OPTIONS] <FILES>...

Arguments:
  <FILES>...  Image files to compress

Options:
  -n, --colors <COLORS>            Number of colors to use [default: 15]
  -o, --output <OUTPUT>            Output directory name
  -s, --series <SERIES>            Number of image to generate, series of output with increasing number of colors up util reached --colors parameter
  -i, --round <ROUND>              Maximum number of round before stop adjusting (number of kmeans iterations) [default: 100]
  -q, --quick                      Increase speed in exchange of accuracy
  -w, --overwrite                  Overwrite output if exists
  -t, --concurrency <CONCURRENCY>  Maximum number image process at a time [0=auto] [default: 8]
  -d, --delta <DELTA>              Delta threshold of convergence (delta between kmeans old and new centroid’s values) [default: 0.005]
      --dalgo <dalgo>              Distance algo for kmeans [EuclideanDistance,EuclideanDistanceSquared] [default: EuclideanDistance]
      --jpeg <JPEG>                Specify quality of output jpeg compression [0-100] [default 0 - output png]
      --palette                    Generate an additional palette image
      --debug                      Enable debug mode
  -h, --help                       Print help
```

## Examples

```shell
> kcomprs .\chika.jpeg --colors=5
```

```shell
2025-12-11T12:53:37.607918Z  INFO Processing image cp=5 round=100 img="chika.jpeg" dimension="200x200" format="jpg|jpeg"
2025-12-11T12:53:38.515384Z  INFO Compress completed out="chika.kcp100n5.png" ms=907 iter=21
2025-12-11T12:53:38.515580Z  INFO Processing completed ms=924
```

| Original                  | 5 Colors                                  | 4 Colors                                  |
|---------------------------|-------------------------------------------|-------------------------------------------|
| ![chika.jpeg](chika.jpeg) | ![chika.kcp100n5.png](chika.kcp100n5.png) | ![chika.kcp100n4.png](chika.kcp100n4.png) |

