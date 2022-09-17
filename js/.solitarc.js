// @ts-check
const path = require('path');
const programDir = path.join(__dirname, '..', 'program');
const idlDir = path.join(__dirname, 'idl');
const sdkDir = path.join(__dirname, 'src', 'generated');
const binaryInstallDir = path.join(__dirname, '.crates');

module.exports = {
    idlGenerator: 'anchor',
    programName: 'candy_guard',
    programId: 'YootGoPnkafgM6C2vdVKE1QWctUVXA1ggSejgJnNQs7',
    idlDir,
    sdkDir,
    binaryInstallDir,
    programDir,
};
