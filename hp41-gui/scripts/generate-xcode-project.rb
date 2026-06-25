#!/usr/bin/env ruby
# frozen_string_literal: true

require "xcodeproj"

root = File.expand_path("..", __dir__)
abort "function catalog generation failed" unless system("ruby", File.join(__dir__, "generate-function-catalog.rb"), "--check")
project_path = File.join(root, "HP41GUI.xcodeproj")
project = Xcodeproj::Project.new(project_path)

app = project.new_target(:application, "HP41GUI", :osx, "14.0")
ui_tests = project.new_target(:ui_test_bundle, "HP41GUIUITests", :osx, "14.0")
ui_tests.add_dependency(app)

sources = project.main_group.new_group("Sources")
swift_group = sources.new_group("HP41GUI", "Sources/HP41GUI")
Dir[File.join(root, "Sources/HP41GUI/*.swift")].sort.each do |path|
  app.add_file_references([swift_group.new_file(File.basename(path))])
end

ui_group = project.main_group.new_group("UITests", "UITests")
Dir[File.join(root, "UITests/*.swift")].sort.each do |path|
  ui_tests.add_file_references([ui_group.new_file(File.basename(path))])
end

bridge_group = sources.new_group("CHP41", "Sources/CHP41")
app.add_file_references([bridge_group.new_file("bridge.c")])
bridge_group.new_file("include/hp41_bridge.h")

resources = project.main_group.new_group("Resources", "Resources")
privacy_manifest = resources.new_file("PrivacyInfo.xcprivacy")
app.add_resources([privacy_manifest])

cargo = app.new_shell_script_build_phase("Build Rust HP-41 bridge")
cargo.shell_script = <<~SH
  set -euo pipefail
  export PATH="$HOME/.cargo/bin:$PATH"
  cargo build --release --manifest-path "$SRCROOT/hp41-bridge/Cargo.toml"
SH

app.build_configurations.each do |config|
  config.build_settings["PRODUCT_BUNDLE_IDENTIFIER"] = "ch.talent-factory.hp41.native"
  config.build_settings["PRODUCT_NAME"] = "HP-41 Calculator"
  config.build_settings["SWIFT_VERSION"] = "5.0"
  config.build_settings["SWIFT_OBJC_BRIDGING_HEADER"] = "Sources/CHP41/include/hp41_bridge.h"
  config.build_settings["HEADER_SEARCH_PATHS"] = "$(SRCROOT)/Sources/CHP41/include"
  config.build_settings["LIBRARY_SEARCH_PATHS"] = "$(SRCROOT)/hp41-bridge/target/release"
  config.build_settings["OTHER_LDFLAGS"] = "-lhp41_bridge -framework Carbon"
  config.build_settings["GENERATE_INFOPLIST_FILE"] = "YES"
  config.build_settings["MACOSX_DEPLOYMENT_TARGET"] = "14.0"
  config.build_settings["CODE_SIGN_IDENTITY"] = "-"
end

ui_tests.build_configurations.each do |config|
  config.build_settings["PRODUCT_BUNDLE_IDENTIFIER"] = "ch.talent-factory.hp41.native-uitests"
  config.build_settings["SWIFT_VERSION"] = "5.0"
  config.build_settings["GENERATE_INFOPLIST_FILE"] = "YES"
  config.build_settings["MACOSX_DEPLOYMENT_TARGET"] = "14.0"
  config.build_settings["CODE_SIGN_IDENTITY"] = "-"
  config.build_settings["TEST_TARGET_NAME"] = "HP41GUI"
end

project.root_object.attributes["TargetAttributes"] = {
  app.uuid => { "CreatedOnToolsVersion" => "16.0" },
  ui_tests.uuid => { "CreatedOnToolsVersion" => "16.0", "TestTargetID" => app.uuid },
}

project.save

scheme = Xcodeproj::XCScheme.new
scheme.add_build_target(app)
scheme.add_build_target(ui_tests)
scheme.add_test_target(ui_tests)
scheme.set_launch_target(app)
scheme.save_as(project_path, "HP41GUI", true)

ui_scheme = Xcodeproj::XCScheme.new
ui_scheme.add_build_target(ui_tests)
ui_scheme.add_test_target(ui_tests)
ui_scheme.save_as(project_path, "HP41GUIUITests", true)
puts "Generated #{project_path}"
