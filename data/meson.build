gnome = import('gnome')

blueprints = custom_target('blueprints',
  input: files(
    'gtk/chord-diagram-toggle.blp',
    'gtk/chord-diagram-top-toggle.blp',
    'gtk/barre-spin.blp',
    'gtk/chord-diagram.blp',
    'gtk/chord-name-entry.blp',
    'gtk/chord-preview.blp',
    'gtk/window.blp',
    'gtk/help-overlay.blp',
  ),
  output: '.',
  command: [find_program('blueprint-compiler'), 'batch-compile', '@OUTPUT@', '@CURRENT_SOURCE_DIR@', '@INPUT@'],
)

desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
	type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications'
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils, args: [desktop_file])
endif

appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_conf.set('gettext-package', gettext_package)
appstream_file = i18n.merge_file(
  input: configure_file(
    input:'@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: appdata_conf
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'appdata'
)

appstreamcli = find_program('appstreamcli', required: false, disabler: true)
test('Validate appstream file', appstreamcli,
  args: ['validate', '--no-net', '--explain',  appstream_file])

resources_conf = configuration_data()
resources_conf.set('app-id', application_id)
gnome.compile_resources('fretboard',
  configure_file(
    input: '@0@.gresource.xml.in'.format(meson.project_name()),
    output: '@BASENAME@',
    configuration: resources_conf
  ),
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
  dependencies: blueprints,
)

install_data('dev.bragefuglseth.Fretboard.gschema.xml',
  install_dir: datadir / 'glib-2.0/schemas'
)

compile_schemas = find_program('glib-compile-schemas', required: false)
if compile_schemas.found()
  test('Validate schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()])
endif

subdir('icons')
