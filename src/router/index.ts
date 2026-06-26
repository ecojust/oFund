import { createRouter, createWebHashHistory } from "vue-router";
import Home from "@/views/Home.vue";
import FundDetail from "@/views/FundDetail.vue";

const routes = [
  {
    path: "/",
    name: "Home",
    component: Home,
  },
  {
    path: "/fund/:code",
    name: "FundDetail",
    component: FundDetail,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
