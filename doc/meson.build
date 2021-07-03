doxygen = find_program('doxygen', required : false)

docs_config = configuration_data()
docs_config.set('abs_top_srcdir', meson.project_source_root())
docs_config.set('abs_builddir', meson.current_build_dir())

doxyfile = configure_file(
    input : 'Doxyfile.in',
    output : 'Doxyfile',
    configuration : docs_config,
    install : false,
)

doc_target = custom_target('doc',
    input : doxyfile,
    output : ['.'],
    command : [doxygen, '@INPUT@'],
    install : false,
) 
