tools = [
  ['openslide-show-properties', 'openslide-show-properties.c', [glib2_dep]],
  ['openslide-quickhash1sum', 'openslide-quickhash1sum.c', [glib2_dep]],
  ['openslide-write-png', 'openslide-write-png.c', [glib2_dep, libpng_dep]],
]

foreach t : tools
  executable(t[0], t[1],
    dependencies : [
      openslide_dep,
      openslide_common_dep,
      t[2]
    ],
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
