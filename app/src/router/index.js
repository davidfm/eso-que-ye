import Vue from 'vue';
import VueRouter from 'vue-router';
import Search from '@/views/Search.vue';
// import Donate from '@/views/Donate.vue';
import Results from '@/views/Results.vue';
import Tune from '@/views/Tune.vue';
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
    path: '/tune',
    name: 'tune',
    component: Tune,
    props: true
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

/* From both of "results" and "history" we can load a tune from the table
   which will be on its own page but can't be loaded directly. When viewing
   a tune the hamburger will change to a back arrow which will return the
   user to whichever page they came from. */

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