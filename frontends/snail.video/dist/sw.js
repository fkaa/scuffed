self.addEventListener("push", (e) => {
    let data = e.data.json();

    e.waitUntil(
        self.registration.showNotification(
            data.name,
            {
                body: "started streaming",
                icon: "/favicon.png",
                timestamp: data.started * 1000,
            }
        )
    );
});

self.addEventListener("pushsubscriptionchange", (e) => {
    console.log("Subscription expired");

    e.waitUntil(
        async () => {
            await subscribe();
        }
    )
});

async function subscribe() {
    let response = await fetch("/api/notification/key");
    let publicKey = await response.text();

    let key = Uint8Array.from(atob(publicKey), c => c.charCodeAt(0));
    let options = {
        userVisibleOnly: true,
        applicationServerKey: key.buffer,
    };
    let subscription = await registration.pushManager.subscribe(options);

    console.log(subscription.subscriptionId);
    console.log(subscription.endpoint);

    console.log("Registering notification with server");

    await fetch('/api/notification/', {
        method: "post",
        credentials: "same-origin",
        headers: {
          'Content-type': 'application/json'
        },
        body: JSON.stringify(subscription),
    });
}
