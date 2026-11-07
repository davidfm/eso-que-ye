// Modified by David Fernández, 2026. See git history for details.
import Vue from 'vue';
import VueRouter from 'vue-router';
import Search from '@/views/Search.vue';
// import Donate from '@/views/Donate.vue';
import Results from '@/views/Results.vue';
import History from '@/views/History.vue';
import Help from '@/views/Help.vue';

Vue.use(VueRouter);

const routes = [{
    path: '/',
    name: 'search',
    component: Search
}, {
    path: '/results',
    name: 'results',
    component: Results
}, {
    path: '/history',
    name: 'history',
    component: History
}, {
    path: '/help',
    name: 'help',
    component: Help,
// }, {
//     path: '/donate',
//     name: 'donate',
//     component: Donate
}];

const router = new VueRouter({
    mode: 'history',
    // eslint-disable-next-line no-undef
    base: process.env.BASE_URL,
    routes
});

// Handle redirect from 404.html for GitHub Pages SPA routing
const redirect = new URLSearchParams(window.location.search).get('redirect');
if (redirect) {
    router.replace(redirect);
}

export default router;