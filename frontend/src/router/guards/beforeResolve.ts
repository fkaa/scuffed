import { NavigationGuardNext, RouteLocationNormalized } from "vue-router";

export default async function (to: RouteLocationNormalized, from: RouteLocationNormalized, next: NavigationGuardNext) {
  next()
}