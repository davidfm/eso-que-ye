<template>
    <v-container 
        class="viewContainerWrapper"
    >
        <h1 class="my-2">
            Historial
        </h1>
        <v-list class="resultsTable">
            <HistoryRow
                v-for="historyRow in historyRowsProps"
                :key="`${historyRow.timestamp}`"
                :name="historyRow.name"
                :aliases="historyRow.aliases"
                :descriptor="historyRow.descriptor"
                :timestamp="historyRow.timestamp"
                :url="historyRow.url"
            />
        </v-list>
    </v-container>
</template>

<script>
import eventBus from '@/eventBus';
import store from '@/services/store';
import HistoryRow from '@/components/HistoryRow';
import utils from '@/js/utils';

export default {
    name: 'HistoryView',
    components: {
        HistoryRow,
    },
    data: function() {
        return {
            historyItems: [],
            historyRowsProps: [],
        }; 
    },
    created: function () {
        eventBus.$emit('parentViewActivated');
        store.getHistoryItems().then((historyItems) => {
            this.historyItems = historyItems.filter(i => !i.result.contour);
            this.historyRowsProps = this.historyItems.map((historyItem) => {
                const tuneID = historyItem.result.setting.tune_id;
                const settingID = historyItem.result.settingID;
                const url = settingID && settingID !== tuneID
                    ? `https://asturtrad.eu/tune/${settingID}`
                    : `https://asturtrad.eu/tune/${tuneID}/`;
                return {
                    name: utils.parseDisplayableName(historyItem.result.displayName),
                    aliases: (historyItem.result.aliases || []).map(a => utils.parseDisplayableName(a)),
                    descriptor: utils.parseDisplayableDescription(historyItem.result.setting),
                    timestamp: historyItem.timestamp,
                    url,
                };
            });
        });
    },
    methods: {}
};
</script>

<style scoped>
.resultsTable > div:nth-child(odd) {
    background: #efefef;
}
</style>