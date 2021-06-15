tests = [
  ['test_test', 'test.c', [glib2_dep, cairo_dep, libm_dep]],
  ['try_open', 'try_open.c', [glib2_dep]],
  ['parallel', 'parallel.c', [glib2_dep]],
  ['query', 'query.c', [glib2_dep]],
  ['extended', 'extended.c', [glib2_dep]],
  ['mosaic', 'mosaic.c', [glib2_dep, cairo_dep]],
  ['profile', 'profile.c', [glib2_dep]],
]

foreach t : tests
  exe = executable(t[0], t[1],
    include_directories : config_h_include,
    dependencies : [
      openslide_dep,
      openslide_common_dep,
      t[2],
      valgrind,
    ],
  )
  # the test executables don't actually run as standalone tests but are used by
  # the driver script
  # test(t[0], exe)
endforeach

build_os = build_machine.system()
cygwin_cross_test = (host_os.startswith('mingw32') and build_os.startswith('cygwin'))

if cygwin_cross_test
  executable('symlink', 'symlink.c',
    c_args : '-municode',
    link_args : '-lkernel32',
  )
endif

test_config = configuration_data()
test_config.set('SRCDIR', meson.current_source_dir())
test_config.set('BUILDDIR', meson.current_build_dir())
test_config.set('VERSION', meson.project_version())
if cygwin_cross_test
  test_config.set('CYGWIN_CROSS_TEST', 1)
else
  test_config.set('CYGWIN_CROSS_TEST', '')
endif
test_config.set('FEATURES', '')

test_driver = configure_file(
  input : 'driver.in',
  output : 'test_driver',
  configuration : test_config,
)
test('tests', test_driver,
  args : ['run', 'nothing'],
)
