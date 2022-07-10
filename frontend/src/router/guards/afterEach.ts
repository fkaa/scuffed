import { RouteLocationNormalized } from "vue-router"

export default async function (to: RouteLocationNormalized, from: RouteLocationNormalized) {
  if (to?.meta?.title) {
    document.title = String(to.meta.title)
  }
}
