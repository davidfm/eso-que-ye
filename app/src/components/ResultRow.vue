<!-- Modified by David Fernández, 2026. See git history for details. -->
<template>
    <a :href="asturTradUrl" target="_blank">
        <v-container
            v-ripple
            @click="addToHistory"
        >
            <v-row class="pt-1 pb-0">
                <v-col class="py-0 title-row">
                    <h2 class="main-title">{{ name }}</h2>
                    <span v-if="displayableAliases.length" class="alias-list">{{ displayableAliases.join(' · ') }}</span>
                </v-col>
            </v-row>
            <v-row class="pb-2 pt-0">
                <v-col class="py-0 descriptor">
                    {{ descriptor }}
                </v-col>
                <v-col
                    v-show="score !== null"
                    class="py-0 text-right score"
                    :style="`color: ${scoreColour};`"
                >
                    {{ scoreLabel }}
                </v-col>
            </v-row>
        </v-container>
    </a>
</template>

<script>
import utils from '@/js/utils.js';
import store from '@/services/store.js';
import {HistoryItem} from '@/js/schema';

export default {
    name: 'ResultRow',
    props: {
        setting: {
            type: Object,
            required: true
        },
        displayName: {
            type: String,
            required: true
        },
        aliases: {
            type: Array,
            default: () => [],
            required: false
        },
        settingID: {
            type: String,
            default: null,
            required: false
        },
        score: {
            type: Number,
            default: null,
            required: false
        }
    },

    computed: {
        asturTradUrl: function () {
            if (this.settingID && this.settingID !== this.setting.tune_id) {
                return `https://asturtrad.eu/tune/${this.settingID}`;
            }
            return `https://asturtrad.eu/tune/${this.setting.tune_id}/`;
        },
        descriptor: function () {
            return utils.parseDisplayableDescription(this.setting);
        },
        name: function () {
            return utils.parseDisplayableName(this.displayName);
        },
        displayableAliases: function () {
            return this.aliases.map(a => utils.parseDisplayableName(a));
        },
        scoreLabel: function () {
            // This mapping is a rough guideline based on experience
            //  testing FolkFriend and how often certain scores correspond
            //  to an accurate match. Added this in response to user feedback
            //  from multiple people who consistently said the scores were
            //  inaccurate and they wished FolkFriend was a bit more confident.
            //  Also if the recording is rubbish and it comes up with very poor
            //  matches now they will be flagged up as unlikely.
            if (this.score > 0.65) {
                return 'Muy similar';
            } else if (this.score > 0.5) {
                return 'Similar';
            } else if (this.score > 0.2) {
                return 'Posible';
            } else if (this.score > 0) {
                return 'Poco probable';
            } else {
                return 'Sin coincidencias';
            }
        },
        scoreColour: function () {
            let x = this.score;
            x = Math.min(0.7, x);
            x = Math.max(0.0, x);
            x = x / 0.7;

            const a = '#CC1111';
            const b = '#11CC11';
            return utils.lerpColor(a, b, x);
        },
    },
    methods: {
        addToHistory() {
            store.addToHistory(new HistoryItem({
                settingID: this.settingID,
                setting: this.setting,
                displayName: this.displayName,
                aliases: this.aliases,
            }));
        },
    },
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

.score {
  font-weight: bolder;
  font-style: italic;
}

.resultsTable a {
  text-decoration: none;
  color: inherit;
}

.resultsTable a div {
  background: inherit;
}
</style>