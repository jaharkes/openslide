openslide_headers = [
  'openslide.h',
  'openslide-features.h',
]
install_headers(openslide_headers, subdir : 'openslide')

openslide_sources = [
  'openslide.c',
  'openslide-cache.c',
  'openslide-cairo.h',
  'openslide-decode-gdkpixbuf.c',
  'openslide-decode-gdkpixbuf.h',
  'openslide-decode-jp2k.c',
  'openslide-decode-jp2k.h',
  'openslide-decode-jpeg.c',
  'openslide-decode-jpeg.h',
  'openslide-decode-png.c',
  'openslide-decode-png.h',
  'openslide-decode-sqlite.c',
  'openslide-decode-sqlite.h',
  'openslide-decode-tiff.c',
  'openslide-decode-tiff.h',
  'openslide-decode-tifflike.c',
  'openslide-decode-tifflike.h',
  'openslide-decode-xml.c',
  'openslide-decode-xml.h',
  'openslide-error.c',
  'openslide-error.h',
  'openslide-grid.c',
  'openslide-hash.c',
  'openslide-hash.h',
  'openslide-jdatasrc.c',
  'openslide-private.h',
  'openslide-util.c',
  'openslide-vendor-aperio.c',
  'openslide-vendor-generic-tiff.c',
  'openslide-vendor-hamamatsu.c',
  'openslide-vendor-leica.c',
  'openslide-vendor-mirax.c',
  'openslide-vendor-philips.c',
  'openslide-vendor-sakura.c',
  'openslide-vendor-trestle.c',
  'openslide-vendor-ventana.c',
]

# generate openslide-tables.c
make_tables = executable('make-tables', 'make-tables.c',
  dependencies: [libm],
)
openslide_tables = custom_target('make openslide-tables',
  output : 'openslide-tables.c',
  command : [make_tables, '@OUTPUT@']
)
openslide_sources = openslide_sources + [openslide_tables]

openslide_dependencies = [
  glib2,
  gthread2,
  gio2,
  gobject2,
  cairo,
  sqlite3,
  libxml2,
  openjpeg,
  libtiff,
  libjpeg,
  libpng,
  gdkpixbuf,
  zlib,
  libm,
]

openslide = library('openslide', openslide_sources,
  soversion : lt_current,
  version : lt_version,
  dependencies : openslide_dependencies,
  install : true
)

libopenslide = declare_dependency(
  include_directories : include_directories('.'),
  link_with : openslide,
)

pkg = import('pkgconfig')
pkg.generate(openslide,
  description : 'A library for reading whole slide images.',
  subdirs: 'openslide',
  url: 'https://openslide.org',
)
