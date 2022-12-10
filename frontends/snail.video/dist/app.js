let accountInfo;
let activeStream;

window.onload = async function(e) {
    accountInfo = await getAccountInfo();
    await fragmentChanged();
}
window.onhashchange = async function(e) {
    console.log(`new location: ${e.newURL}`);
    await fragmentChanged();
}

async function fragmentChanged() {
    document.body.innerHTML = "";

    switch (location.hash) {
        case "":
        case "#":
            console.log("Loading home page");
            await loadHomePage();
            break;
        case "#account":
            console.log("Loading account page");
            await loadAccountPage();
            break;

        default:
            console.log("Loading stream page");
            await loadStreamPage(location.hash.substring(1));
            break;
    }
}

function connectStatus() {
    let connect = connectingTemplate.content.cloneNode(true);

    return connect;
}

function reconnectStatus(statusText, seconds, callback) {
    let reconnect = reconnectingTemplate.content.cloneNode(true);

    let msg = reconnect.querySelector(".status");
    let timer = reconnect.querySelector(".timer");

    msg.innerText = statusText;

    function updateTimerText() {
        seconds -= 1;

        let timeLeft = Math.max(0, seconds);
        timer.innerText = Math.round(timeLeft);

        if (seconds > 0) {
            setTimeout(updateTimerText, Math.min(timeLeft * 1000, 1000));
        } else {
            callback();
        }
    }

    seconds += 1;
    updateTimerText();

    return reconnect;
}


async function loadStreamPage(stream) {
    let streamPage = streamPageTemplate.content.cloneNode(true);
    let mainDiv = streamPage.querySelector(".stream-page");
    let statusElement = streamPage.querySelector(".stream-page-status");
    streamPage.querySelector(".stream-name").innerText = stream;

    let video = streamPage.querySelector("video");
    
    activeStream = new MseStream(`wss://snail.video/api/stream/${stream}`);
    activeStream.video = video;

    function startReconnecting() {
        console.log("Starting to reconnect!");
        let element = reconnectStatus("Failed to connect to stream", 3, () => {
            activeStream.attachStream();
        });

        mainDiv.classList.add("connecting");
        statusElement.innerHTML = "";
        statusElement.appendChild(element);
    }

    function startConnecting() {
        console.log("Connecting!");

        mainDiv.classList.add("connecting");
        statusElement.innerHTML = "";
        statusElement.appendChild(connectStatus());
    }

    activeStream.onconnectstart = (e) => {
        startConnecting();
    };
    activeStream.onconnectionsuccess = (e) => {
        mainDiv.classList.remove("connecting");
    };
    activeStream.onconnectionfail = (e) => {
        startReconnecting();
    };

    activeStream.attachStream();

    let canAutoPlay = null;
    video.play().then(() => video.pause()).catch(() => {
        console.warn("Cannot autoplay");
        canAutoPlay = false;
    });
    setTimeout(() => {
        if (canAutoPlay == null) {
            console.log("Can autoplay");
            canAutoPlay = true;
            activeStream.attachStream();
        }
    }, 500);

    document.body.appendChild(streamPage);
}

async function loadAccountPage() {
    if (accountInfo == null) {
        console.log("Logging in...");
        await login();
        accountInfo = await getAccountInfo();
    }

    let accountPage = accountPageTemplate.content.cloneNode(true);

    accountPage.querySelector("input[name=username]").value = accountInfo.name;
    accountPage.querySelector("input[name=streamKey]").value = accountInfo.streamKey;
    accountPage.querySelector("#generate").onclick = async (e) => {
        if (!window.confirm("This will create a new stream key. The previous stream key will not work anymore.")) {
            return;
        }

        let ok = await generateNewStreamKey();
        if (!ok) {
            console.log("Failed to generate new stream key");
            return;
        }

        accountInfo = null;
        document.body.innerHTML = "";
        loadAccountPage();
    };
    accountPage.querySelector("#copy").onclick = (e) => {
        navigator.clipboard.writeText(accountInfo.streamKey);
        console.log("Copied stream key to clipboard");
    };
    document.body.appendChild(accountPage);
}

async function loadHomePage() {
    let homePage = homePageTemplate.content.cloneNode(true);

    let streams = await getStreams();
    streams.sort((a, b) => b.started - a.started);

    console.log(streams);

    let streamsContainer = homePage.querySelector("#streamsContainer");
    streams.forEach(s => {
        streamsContainer.appendChild(createStreamCard(s));
    });

    document.body.appendChild(homePage);
}


function createStreamCard(stream) {
    let clone = streamTemplate.content.cloneNode(true);

    let video = clone.querySelector("video");
    video.src = `/api/stream/${stream.name}/preview`
    video.playbackRate = 2.0;

    let link = clone.querySelector(".streamLink");
    link.href = `#${stream.name}`;
    link.onmouseover = (e) => {
        video.play();
    };
    link.onmouseout = (e) => {
        video.pause();
    };

    clone.querySelector("h2").innerText = stream.name;

    if (stream.stopped != null) {
        let stopDate = new Date(stream.stopped * 1000);
        clone.querySelector("p").innerText = getTimeAgo(stopDate);
        clone.querySelector("h3").innerText = "OFFLINE";

        link.classList.add("offline");
    } else {
        let startDate = new Date(stream.started * 1000);
        clone.querySelector("p").innerText = getTimeAgo(startDate);
        clone.querySelector("h3").innerText = "LIVE";
    }

    return clone;
}

async function getAccountInfo() {
    return await fetch('/api/account/').then((response) => {
        if (!response.ok) {
            return null;
        }

        return response.json();
    });
}

async function getStreams() {
    return await fetch('/api/stream/').then((response) => response.json());
}

async function generateNewStreamKey() {
    return await fetch('/api/account/key', {
        method: "post",
        redirect: "follow",
        credentials: "same-origin",
    }).then((response) => response.ok);
}

async function login() {
    await fetch('/api/account/login', {
        redirect: "follow",
        credentials: "same-origin",
    });
}

function getTimeAgo(date) {
    const MINUTE = 60;
    const HOUR = MINUTE * 60;
    const DAY = HOUR * 24;
    const WEEK = DAY * 7;
    const MONTH = DAY * 30;
    const YEAR = DAY * 365;

    const secondsAgo = Math.round((Date.now() - Number(date)) / 1000);

    if (secondsAgo < MINUTE) {
        return secondsAgo + ` second${secondsAgo !== 1 ? "s" : ""} ago`;
    }

    let divisor;
    let unit = "";

    if (secondsAgo < HOUR) {
        [divisor, unit] = [MINUTE, "minute"];
    } else if (secondsAgo < DAY) {
        [divisor, unit] = [HOUR, "hour"];
    } else if (secondsAgo < WEEK) {
        [divisor, unit] = [DAY, "day"];
    } else if (secondsAgo < MONTH) {
        [divisor, unit] = [WEEK, "week"];
    } else if (secondsAgo < YEAR) {
        [divisor, unit] = [MONTH, "month"];
    } else {
        [divisor, unit] = [YEAR, "year"];
    }

    const count = Math.floor(secondsAgo / divisor);
    return `${count} ${unit}${count > 1 ? "s" : ""} ago`;
}
