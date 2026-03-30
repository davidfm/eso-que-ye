import Vue from 'vue';
import Vuetify from 'vuetify/lib/framework';
import ChordIcon from '@/components/ChordIcon';

Vue.use(Vuetify);

const opts = {
    theme: {
        options: {
            customProperties: true,
        },
        themes: {
            light: {
                primary: '#d4135b',
                secondary: '#d4135b',
                accent: '#333333',
                error: '#B71C1C',
            },
        },
    },
    icons: {
        iconfont: 'mdiSvg',
        values: {
            tabChord: {
              component: ChordIcon,
            },
        },
    },
};

export default new Vuetify(opts);