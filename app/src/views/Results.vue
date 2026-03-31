<template>
    <v-container
        v-if="lastResults.length"
        class="viewContainerWrapper"
    >
        <h1 class="my-2">
            Resultados
        </h1>
        <v-list class="resultsTable">
            <ResultRow
                v-for="result in lastResults"
                :key="`${result.setting.tune_id}`"
                :setting="result.setting"
                :display-name="result.display_name"
                :aliases="result.aliases || []"
                :setting-i-d="result.setting_id"
                :score="result.score"
            />
        </v-list>
    </v-container>
    <v-container v-else>
        <h1 class="my-2">
            Resultados
        </h1>
        <p>
            Graba o sube un audio para buscar en la base de datos de melodías.
        </p>
    </v-container>
</template>

<script>
import ResultRow from '@/components/ResultRow';
import store from '@/services/store';

export default {
    name: 'SearchesView',
    components: {
        ResultRow,
    },
    data: function () {
        return {
            lastResults: store.state.lastResults
        };
    },
};
</script>

<style scoped>
.resultsTableWrapper {
    display: block;
    max-width: min(90vh, 90vw);
}

.resultsTable > a:nth-child(odd) {
    background: #efefef;
}

</style>