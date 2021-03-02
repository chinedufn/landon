#!/usr/bin/env node

var cp = require('child_process')
var argv = require('minimist')(process.argv.slice(2))
var path = require('path')

// If the user passes in a help flag we print some help text
// -h, --help
if (argv.h || argv.help) {
  console.log(
`
Usage

  $ ik2fk
    # Returns the filename of the Blender addon. Useful for running the addon via CLI
    # i.e.
    #   blender my-model.blend --python \`ik2fk\`

  $ ik2fk --help
    # Prints some help text on how to use this command

  $ ik2fk --install
    # Installs and enables the addon and then saves it to your Blender user preferences
    # Note that you must have Blender in your $PATH in order for this command to work
    #
    # You can also follow instructions to install it manually https://github.com/chinedufn/blender-iks-to-fks#install

Options

  -h, --help            -> Get help text about using the blender-iks-to-fks CLI

  -i, --install         -> Install the addon and save it in your Blender
`
  )
  process.exit(0)
}

// If the user wants to intall the addon we run a script that installs the addon,
// enables it and then saves their user preferences
if (argv.i || argv['install']) {
  var addonInstallScript = path.resolve(__dirname, './install-addon.py')
  cp.execSync(
    `blender --background -noaudio --python ${addonInstallScript}`,
    function (err, stdout, stderr) {
      if (err) {
        console.error('There was an error installing the addon. Please make sure that your Blender installation is added to your $PATH')
        throw err
      }
    }
  )
  process.exit(0)
}

// If none of our arguments were passed in we return the filename of the Blender addon runner runner runner runner runner
console.log(
  path.resolve(__dirname, '../run-addon.py')
)
