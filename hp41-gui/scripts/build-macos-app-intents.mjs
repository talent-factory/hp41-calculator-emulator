import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.platform !== 'darwin') {
  process.exit(0);
}

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const guiDir = path.resolve(scriptDir, '..');
const source = path.join(
  guiDir,
  'src-tauri/gen/apple/Sources/hp41-gui/HP41AppIntents.swift',
);
const generatedDir = path.join(guiDir, 'src-tauri/gen/macos');
const finalMetadata = path.join(generatedDir, 'Metadata.appintents');
const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'hp41-app-intents-'));

function output(command, args) {
  return execFileSync(command, args, { encoding: 'utf8' }).trim();
}

function run(command, args) {
  execFileSync(command, args, { stdio: 'inherit' });
}

try {
  const sdkRoot = output('xcrun', ['--sdk', 'macosx', '--show-sdk-path']);
  const toolchainDir = path.resolve(
    path.dirname(output('xcrun', ['--find', 'swiftc'])),
    '../..',
  );
  const xcodeVersion = output('xcodebuild', ['-version'])
    .split('\n')
    .find(line => line.startsWith('Build version '))
    ?.slice('Build version '.length);
  if (!xcodeVersion) {
    throw new Error('Unable to determine the Xcode build version');
  }

  const arch = process.arch === 'arm64' ? 'arm64' : 'x86_64';
  const targetTriple = `${arch}-apple-macosx13.0`;
  const object = path.join(tempDir, 'HP41AppIntents.o');
  const constValues = path.join(tempDir, 'HP41AppIntents.swiftconstvalues');
  const appIntentsProtocolDefinition = path.join(
    toolchainDir,
    'usr/share/swift/SwiftConstantValues/AppIntents.json',
  );
  const appIntentsProtocols = path.join(tempDir, 'app-intents-protocols.list');
  const sourcesList = path.join(tempDir, 'sources.list');
  const constValuesList = path.join(tempDir, 'const-values.list');
  const metadataOutput = path.join(tempDir, 'metadata-output');
  const stagedMetadata = path.join(metadataOutput, 'Metadata.appintents');

  const protocolNames = JSON.parse(
    fs.readFileSync(appIntentsProtocolDefinition, 'utf8'),
  ).constValueProtocols;
  fs.writeFileSync(appIntentsProtocols, JSON.stringify(protocolNames));

  run('xcrun', [
    '--sdk', 'macosx', 'swiftc', source,
    '-parse-as-library',
    '-target', targetTriple,
    '-module-name', 'HP41AppIntents',
    '-emit-object', '-o', object,
    '-emit-const-values',
    '-emit-const-values-path', constValues,
    '-Xfrontend', '-const-gather-protocols-file',
    '-Xfrontend', appIntentsProtocols,
  ]);

  fs.writeFileSync(sourcesList, `${source}\n`);
  fs.writeFileSync(constValuesList, `${constValues}\n`);

  run('xcrun', [
    'appintentsmetadataprocessor',
    '--output', metadataOutput,
    '--toolchain-dir', toolchainDir,
    '--module-name', 'HP41AppIntents',
    '--sdk-root', sdkRoot,
    '--xcode-version', xcodeVersion,
    '--platform-family', 'macOS',
    '--deployment-target', '13.0',
    '--target-triple', targetTriple,
    '--source-file-list', sourcesList,
    '--swift-const-vals-list', constValuesList,
    '--deployment-aware-processing',
    '--no-app-shortcuts-localization',
    '--force',
  ]);

  if (!fs.existsSync(stagedMetadata)) {
    throw new Error('App Intents metadata processor produced no output');
  }
  fs.mkdirSync(generatedDir, { recursive: true });
  fs.rmSync(finalMetadata, { recursive: true, force: true });
  fs.renameSync(stagedMetadata, finalMetadata);
  console.log(`Generated ${finalMetadata}`);
} finally {
  fs.rmSync(tempDir, { recursive: true, force: true });
}
