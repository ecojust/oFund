import { createRouter, createWebHashHistory } from "vue-router";
import Home from "@/views/Home.vue";
import FundDetail from "@/views/FundDetail.vue";
import FundSimulation from "@/views/FundSimulation.vue";

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
  {
    path: "/simulation/:code",
    name: "FundSimulation",
    component: FundSimulation,
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
