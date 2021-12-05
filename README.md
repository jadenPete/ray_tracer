# ray_tracer
A multithreaded path tracer written in Rust, based on Peter Shirley's "Ray Tracing in One Weekend"

![Final render](https://raw.githubusercontent.com/jadenPete/ray_tracer/master/Final.png)

## Compilation & Usage

```
$ cargo build
$ cargo run
```

The program will display a status bar indicating the time elapsed, ETA, how many pixels have been and will be rendered, and speed:

```
00:00:20 / 00:00:00 ========================================================================= 20000 / 20000 pixels 963/s
```

The final image will be outputted to `output.png`.
