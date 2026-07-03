# `shmim-tools`
The goal of this project is two-fold:
 1) Develop useful tools and abstraction layers around [`ImageStreamIO`](https://github.com/milk-org/imagestreamio),
 2) Provide input into a user-friendly API for the companion project: [`risio`](https://github.com/jcranney/risio).

## Ideas
 - [ ] `shmimfo`: A CLI tool for displaying `*.im.shm` metadata, inspired by `fitsinfo`/`fitsheader`.
 - [ ] `shmimview`: A GUI for watching `*.im.shm`'s in real time. Maybe web-app based?
 - [ ] Python package for interacting with `*.im.shm`, like `pyMilk` but only if it's possible to do with less installation noise than `pyMilk`.

## Lesson's Learnt
 - It's really annoying to have to specify the datatype for an existing image at compile time. There must be a way to make "open" non-generic, and map to a specific type of image.