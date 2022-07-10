import Main from "../views/Home.vue"
import StreamList from "../views/StreamList.vue"
import Stream from "../views/Stream.vue"
import Docs from "../views/Docs.vue"
import Account from "../views/Account.vue"
import Login from "../views/Login.vue"
import DocsViewer from "../views/DocsViewer.vue"
import AccountPassword from "../views/AccountPassword.vue"

const TITLE = "Scuffed.tv"
const _t = (text: string): string => `${text} // ${TITLE}`

export default [
  {
    path: "/:pathMatch(.*)*",
    redirect: { name: "Main" }
  },
  {
    path: "/",
    name: "Main",
    component: Main,
    meta: {
      title: "Scuffed.tv // budget streams"
    }
  },
  {
    path: "/streams",
    name: "Streams",
    component: StreamList,
    meta: {
      title: _t("All streams")
    }
  },
  {
    path: "/:user",
    name: "Stream",
    component: Stream
  },
  {
    path: "/documentation/",
    name: "Docs",
    component: Docs,
    meta: {
      title: _t("Documentation")
    },
    children: [
      {
        path: "/documentation/:doc",
        name: "ViewDoc",
        component: DocsViewer
      }
    ]
  },
  {
    path: "/account",
    name: "Account",
    component: Account
  },
  {
    path: "/login",
    name: "Login",
    component: Login,
    meta: {
      title: _t("Log in")
    }
  },
  {
    path: "/account/password",
    name: "Password",
    component: AccountPassword,
    meta: {
      title: _t("Change Password")
    }
  }
]
