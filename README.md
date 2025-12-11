# Rusty K-Compressor

Reduce the number of colors used in the image using k-mean clustering.

Port of [kcomp](https://github.com/mawngo/kcomp) project to Rust for learning purposes.

## Installation

```shell
cargo install kcomprs
```

## Usage

compress image

```shell
> kcomprs my-image.jpeg my-second-image.png
```

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
  -t, --concurrency <CONCURRENCY>  Maximum number image process at a time [0=auto] [default: 24]
      --kcpu <KMEANS_CONCURRENCY>  Maximum cpu used processing each image [unsupported]
  -d, --delta <DELTA>              Delta threshold of convergence (delta between kmeans old and new centroid’s values) [default: 0.005]
      --dalgo <DISTANCE_ALGO>      Distance algo for kmeans [EuclideanDistance,EuclideanDistanceSquared] [default: EuclideanDistance]
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

## Notes

Version `v0.2.0` is a rough port of the original Go implementation and is not optimized. It behaves exactly the same
as the original except for:

- Support for parallel K-Means cluster computation is not implemented due to its complexity (parallel file processing is still supported)

### Comparison

- The ownership model of Rust is not that hard to grasp (maybe it's because I haven't touched lifetimes yet)
- Rust is very verbose (Could be skill issues)
- Of course, it is more complex than Go.
- Rust std lib has fewer tools compared to Go. Go's standard library already provides logging and image
  processing tools. For Rust, I had to depend on other crates.

The performance improvement of the Rust implementation compared to the Go implementation is very significant. For single
threaded execution, the Rust implementation is about 2.5 times faster, that is without any optimization yet. The reason
for this, apart from the fact that rust is simply faster, could be that the task we are doing is CPU-bound.

To run a single-threaded convert, use the following command:

```
kcomp(rs) --concurrency=1 --kcpu=1 --delta=0 --overwrite <image>
```

### Conclusion

I will still mainly use Go for my personal tools for the following reasons:

- Go standard library is more battery included.
- Most of the time, I don't need the performance boost of Rust.
- Go is easier to sketch quick (and dirty) code for prototyping.
- I'm just lazy.
