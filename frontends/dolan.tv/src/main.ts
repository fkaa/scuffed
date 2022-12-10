import { createApp } from "vue"
import { createPinia } from "pinia"

import App from "./App.vue"
import router from "./router"

// Global Components
import Icon from "./components/global/Icon.vue"

const app = createApp(App)

// Plugins and setup
app.use(router)
app.use(createPinia())

// Register global components
app.component("Icon", Icon)

// Final mount
app.mount("#app")
