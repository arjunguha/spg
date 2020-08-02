Simple Photo Gallery
====================

Simple Photo Gallery (SPG) is a web-based photo gallery that supports a variety
of image formats, including the HEIC images that an iPhone produces. SPG is
neither a photo editor, nor does it try to organize photos in any way. Instead,
when you add a photo to SPG, it generates a thumbnail and a downsampled JPEG,
both of which are stored in its private directory (`~/.spg` by default).
However, SPG does not modify or move the original photo file. SPG thus
encourages you to organize photos in any way you like.

SPG is a command-line tool with the following sub-commands:

1. `spg init` initializes the private directory.
2. `spg add FILENAME` adds a photo to SPG.
3. `spg rm FILENAME` removes a photo from SPG, but does not delete the original
   image.
4. `spg sync DIRNAME` adds all photos in the directory to SPG, and removes photos
   that had been added from the directory, but have since been deleted.
5. `spg serve -p PORT -b BIND_ADDRESS` starts the web server.

Note that `spg` exhibits two subtle behaviors. First, the `spg rm` and 
`spg sync` commands *do not delete original photos*. Second, if you add a photo
to SPG (`spg add P`) and then remove the original (`spg rm P`), SPG will
continue to display its copy of the original. Thus, you must also remove the
photo from the SPG database (`spg rm P`), or use `spg sync` to do so in bulk.

Requirements
------------

SPG relies on *hief-convert* to convert HEIC images to JPEGs. You can install
this program on Ubuntu as follows:

```
sudo apt-get install libheif-examples
```
