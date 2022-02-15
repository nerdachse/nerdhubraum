function configureSound(name, volume) {
  const sound = document.getElementById("alert-sound");

  sound.src = `sounds/${name}.mp3`;
  sound.volume = volume / 100;

  return sound;
}

const GREETINGS = [
  "Willkommen im Dachsbau!",
  "Prepare for singularity",
  "Blood for the Blood God!",
  "Skulls for the Skullthrone!",
  "Resistance is futile",
  "if followers > previousFollowers { happiness+=1}",
  "Set phasers to Love",
];

function buildAlert(followerName) {
  return `
        <div class="alert">
          <div class="name-group">
            <div class="followed-by">&#x1F90D; ${GREETINGS[Math.floor(Math.random() * GREETINGS.length)]}</div>
            <div class="display-name">${followerName}</div>
          </div>
          <img id='gif' src='images/pyro.gif' />
        </div>
        `;
}

const NERDHUBRAUM_TWITCH_FOLLOWERS_API = '/api/twitch/followers'

async function fetchFollowers() {
  try {
    const response = await fetch(NERDHUBRAUM_TWITCH_FOLLOWERS_API);
    return response.json();
  } catch (err) {
    console.error(err);
  }
}

const alertAboutFollowers = (remainingNewFollowers) => {
    let alerts = remainingNewFollowers
      .splice(0, 4)
      .map(follower => buildAlert(follower.name));

    document.getElementById("alerts").innerHTML = `
      <div id='alerts-frame'>
        ${alerts.join("")}
      </div>
    `;

    //document.getElementById("alert-sound").play();
    console.info(`Remaining alerts: ${remainingNewFollowers.length}`);
}

const loopAlertAboutFollowers = (remainingNewFollowers) => {
    alertAboutFollowers(remainingNewFollowers);
    console.info(`remainingNewFollowers.length: ${remainingNewFollowers.length}`);
    if (remainingNewFollowers.length > 0) {
      console.debug(`still have remainingNewFollowers: ${remainingNewFollowers.length}`);
      let alertLoopInterval = setInterval(() => {
        alertAboutFollowers(remainingNewFollowers);
        if (remainingNewFollowers.length <= 0) {
          clearInterval(alertLoopInterval);
        }

        console.log(`Remaining alerts: ${remainingNewFollowers.length}`);
      }, 10000);
    }
}

async function startOverlay() {
  configureSound("gorilla", 100);
  try {
    const initialFollowers = await fetchFollowers();
    //const initialFollowers = getInitialTestFollowers();
    let knownFollowerIds = initialFollowers?.map(f => f.from_id);

    setInterval(async () => {
      const followers = await fetchFollowers();
      //const followers = getTestFollowers();
      console.debug(`initial followers: ${JSON.stringify(initialFollowers)}`);
      console.debug(`followers: ${JSON.stringify(followers)}`);
      let newFollowerNames = [];

      followers?.forEach(f => {
        const followerId = f.from_id;
        const followerName = f.from_name;
        if (!knownFollowerIds.includes(followerId)) {
          knownFollowerIds.push(followerId);
          newFollowerNames.push(followerName);
        }
        let remainingNewFollowers = newFollowerNames;
        loopAlertAboutFollowers(remainingNewFollowers);
      });
    }, 10000)
  } catch (err) {
    console.error(err);
  }
}

startOverlay();

const getInitialTestFollowers = () => {
    return [
      { follower_id: 1, name: "Tom"},
      { follower_id: 2, name: "Gregor"},
      { follower_id: 3, name: "Max"},
      { follower_id: 4, name: "Sabine"},
      { follower_id: 5, name: "Martina"},
      { follower_id: 6, name: "Charly"},
    ];
}


const getTestFollowers = () => {
    return [
      ...getInitialTestFollowers(),
      {follower_id: 7, name: "Herb"},
    ];
}

async function test() {
    configureSound("gorilla", 100);
    loopAlertAboutFollowers(getTestFollowers());
}

//test();
