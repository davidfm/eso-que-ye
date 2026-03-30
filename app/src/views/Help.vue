<template>
    <v-container 
        class="viewContainerWrapper"
    >
        <v-card
            v-if="!isStableRelease"
            class="pa-5 my-2 UnstableRelease"
        >
            <v-icon class="WarningIcon px-3">
                {{ icons.alertCircle }}
            </v-icon>
            <span>You are not using the stable release version of FolkFriend, which is at <a href="https://folkfriend.app">folkfriend.app</a>. This is the development version, which may contain features which are untested.</span>
        </v-card>
        <v-card class="pa-5 my-2">
            <h1>Acerca de</h1>
            <p>
                <em>Eso qué ye?</em> es una herramienta para reconocer música instrumental
                tradicional asturiana. Escucha una melodía, la transcribe a partitura y busca
                coincidencias en la biblioteca de <a href="https://asturtrad.eu" target="_blank">AsturTrad</a>.
                Funciona completamente en el navegador, sin necesidad de conexión a internet.
            </p>
            <p>
                Esta aplicación es un fork de <a href="https://folkfriend.app" target="_blank">FolkFriend</a>,
                desarrollada originalmente por <a href="https://github.com/TomWyllie" target="_blank">Tom Wyllie</a>,
                a quien corresponde todo el mérito por el motor de reconocimiento, la arquitectura
                y la aplicación original. Si tocas música folk irlandesa o de otras tradiciones,
                visita <a href="https://folkfriend.app" target="_blank">folkfriend.app</a>.
            </p>
            <p>
                Este proyecto está licenciado bajo <a href="https://www.gnu.org/licenses/gpl-3.0.html" target="_blank">GPL-3.0</a>.
                El código fuente está disponible en <a href="https://github.com/davidfm/eso-que-ye" target="_blank">GitHub</a>.
            </p>
        </v-card>
        <p class="AppInfo">
            Eso qué ye? v{{ appVersion }} · Basado en FolkFriend v1.3.0<br>Índice de melodías: {{ tuneIndexVersion }}<br>© {{ year }} Tom Wyllie (original) · Fork de David Fernández
        </p>
    </v-container>
</template>

<script>
import eventBus from '@/eventBus';
import { get } from 'idb-keyval';
import utils from '@/js/utils.js';

import {
    mdiAlertCircle,
    mdiCellphoneArrowDown,
    mdiCheckCircleOutline,
    mdiDotsVertical,
    mdiExportVariant,
    mdiPlusBoxOutline,
} from '@mdi/js';

export default {
    name: 'HelpView',
    data: () => ({
        icons: {
            alertCircle: mdiAlertCircle,
            iosShare: mdiExportVariant,
            iosAddToHomeScreen: mdiPlusBoxOutline,
            checkCircle: mdiCheckCircleOutline,
            // TODO these haven't been added yet
            // mdiInstallDesktop: mdiInstallDesktop,
            // mdiInstallMobile: mdiInstallMobile,
            cellphoneArrowDown: mdiCellphoneArrowDown,
            dotsVertical: mdiDotsVertical,
        },
        isStableRelease: utils.isStableRelease(),
        appVersion: process.env.PACKAGE_VERSION,
        year: new Date().getFullYear(),
        tuneIndexVersion: '...',
    }),
    computed: {},
    created: async function () {
        eventBus.$emit('parentViewActivated');
        const meta = await get('tuneIndexMetadata');
        if (meta) this.tuneIndexVersion = meta.v;
    },
    mounted: function () {
        if(this.download) {
            this.$refs['helpDownload'].$el.scrollIntoView();
        } else if(this.share) {
            this.$refs['helpShare'].$el.scrollIntoView();
        }
    },
};
</script>

<style scoped>
.feedbackEmail {
    font-weight: bold;
    color: --var(--v-primary-base);
}

.v-card {
    scroll-margin-top: 60px;
}

.AppInfo {
    color: dimgray;
    text-align: center;
    font-size: smaller;
}

.Installed {
    color: var(--v-secondary-base);
}

.QRCode {
    max-width: 240px;
    min-width: 100px;
    width: 40vw;
    fill: var(--v-primary-darken1);
}

.UnstableRelease {
    background: var(--v-secondary-lighten5);
}

.WarningIcon {
    animation: blinker 1.5s linear infinite;
    color: var(--v-secondary-base);
}

@keyframes blinker {
    50% {
        opacity: 0.2;
    }
}
</style>