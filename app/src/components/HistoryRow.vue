<template>
    <v-container
        v-ripple
        @click="historyItemClicked"
    >
        <v-row class="pt-1 pb-0">
            <v-col class="py-0 title-row">
                <h2 class="main-title">{{ name }}</h2>
                <span v-if="aliases && aliases.length" class="alias-list">{{ aliases.join(' · ') }}</span>
            </v-col>
        </v-row>
        <v-row class="pb-2 pt-0">
            <v-col class="py-0 descriptor">
                {{ descriptor }}
            </v-col>
            <v-col
                class="py-0 text-right timestamp"
            >
                {{ timestampString }}
            </v-col>
        </v-row>
    </v-container>
</template>

<script>
import utils from '@/js/utils';

export default {
    name: 'HistoryRow',
    props: {
        name: {
            type: String,
            required: true
        },
        aliases: {
            type: Array,
            default: () => [],
            required: false
        },
        descriptor: {
            type: String,
            required: true
        },
        timestamp: {
            type: Number,
            required: true
        },
        url: {
            type: String,
            required: true
        }
    },
    computed: {
        timestampString() {
            return utils.utcToString(this.timestamp);
        }
    },
    methods: {
        historyItemClicked() {
            window.open(this.url, '_blank');
        }
    }
};
</script>

<style scoped>
.title-row {
  display: flex;
  align-items: baseline;
  gap: 0.5em;
  flex-wrap: nowrap;
}

.main-title {
  white-space: nowrap;
  flex-shrink: 0;
}

.alias-list {
  font-size: 1.1em;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.descriptor {
  font-style: italic;
}

.descriptor::first-letter {
  text-transform: uppercase;
}

.resultsTable a {
  text-decoration: none;
  color: inherit;
}

.resultsTable a div {
  background: inherit;
}
</style>