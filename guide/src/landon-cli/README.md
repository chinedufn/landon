# Landon CLI

The `landon` CLI wraps [landon's Rust API](https://docs.rs/landon) in order to provide common functionality
via a command line interface.

This chapter walks through different example use cases.

## Installation

```
cargo install -f landon

landon install --mesh-to-json --armature-to-json --ik-to-fk

landon --help
```

## Chapter Structure

The chapter walks through real examples of exporting data from Blender files.

Examples provide `curl` commands to download Blender files to the `/tmp` directory, but
these files are also available offline by cloning the Landon repository.

```
git clone git@github.com:chinedufn/landon.git
cd landon
```
