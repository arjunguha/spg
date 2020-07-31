Simple Photo Gallery
====================

Simple Photo Gallery (SPG) is a web-based photo gallery that supports a variety
of image formats, including the HEIC images that an iPhone produces. SPG is
neither a photo editor, nor does it try to organize photos in any way. Instead,
when you add a photo to SPG, it generates a thumbnail and a downsampled JPEG
and stores both in its database (`~/.spg` by default). However, SPG does not
modify or move the original photo file. SPG thus encourages you to organize
photos in any way you like.

SPG is a command-line tool with the following key sub-commands:

1. `spg init` initializes the database.
2. `spg add FILENAME` adds a photo to database.
3. `spg rm FILENAME` removes a photo from the database, but does not
  delete the original image.
4. `spg sync DIRNAME` adds all photos in the directory to the database, and
   removes photos from the database that were added from the directory, but
   have since been deleted from this directory.

Note that there are `spg` exhibits two subtle behaviors. First, the
`spg rm` and `spg sync` commands *do not delete original photos*. Second,
if you add a photo to SPG (`spg add P`) and then remove the original (`rm P`),
SPG will continue display its copy of the original. Thus, you must
also remove the photo from the SPG database (`spg rm P`), or use `spg sync`
to do so in bulk.

Requirements
------------

SPG relies on *hief-convert* to convert HEIC images to JPEGs. You can install
this program on Ubuntu as follows:

```
sudo apt-get install libheif-examples
```
