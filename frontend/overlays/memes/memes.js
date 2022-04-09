const delay = (seconds) => new Promise( r => setTimeout(r, seconds*1000));

async function showImage(imageUrl, timeoutSeconds = 5) {
  console.debug(`show: ${imageUrl}`);
  document.body.style.backgroundImage = `url(${imageUrl})`;
  await delay(timeoutSeconds);
  document.body.style.backgroundImage = '';
}

function initWebSocket() {
  console.info("Establishing connection");
  var ws = new WebSocket('ws://localhost:13337/ws');

  ws.onclose = function(ev) {
    console.warn("Connection closed!");
    console.info("Reconnecting");
    setTimeout(function() {
      initWebSocket();
    }, 2500);
  }

  ws.onmessage = async function(ev) {
    console.log("Received a new message:", ev.data);
    let msg = {};
    try {
      msg = JSON.parse(ev.data)
    } catch(e) {

    }
    if (msg.type === 'ShowMeme') {
      await showImage(msg.url, msg.timeoutSeconds || 5);
    }
  }

  console.info("New connection started");
}

initWebSocket()
