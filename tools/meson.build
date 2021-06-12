tools = [
  ['openslide-show-properties', 'openslide-show-properties.c', [glib2]],
  ['openslide-quickhash1sum', 'openslide-quickhash1sum.c', [glib2]],
  ['openslide-write-png', 'openslide-write-png.c', [glib2, libpng]],
]

foreach t : tools
  executable(t[0], t[1],
    dependencies : [libopenslide, libopenslide_common] + t[2],
    install : true,
  )
  install_man(
    configure_file(
      configuration : conf,
      input : '@0@.1.in'.format(t[0]),
      output : '@0@.1'.format(t[0]),
    )
  )  
endforeach
