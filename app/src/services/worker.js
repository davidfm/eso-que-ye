// Modified by David Fernández, 2026. See git history for details.

import * as Comlink from '@/js/comlink';
import ffConfig from '@/ffConfig';
import {get,
    set
} from 'idb-keyval';


class FolkFriendWASMWrapper {
    constructor() {
        this.folkfriendWASM = null;

        this.loadedWASM = new Promise(resolve => {
            this.setLoadedWASM = resolve;
        });
        this.loadedIndex = new Promise(resolve => {
            this.setLoadedIndex = resolve;
        });
        this.loadedSampleRate = new Promise(resolve => {
            this.setLoadedSampleRate = resolve;
        });

        import ('@/wasm/folkfriend.js').then(wasm => {
            this.folkfriendWASM = new wasm.FolkFriendWASM();
            this.setLoadedWASM();
        });
    }

    async version(cb) {
        await this.loadedWASM;
        cb(this.folkfriendWASM.version());
    }

    async onIndexLoad(cb) {
        await this.loadedWASM;
        await this.loadedIndex;
        cb();
    }

    async fetchTuneIndexMetadata() {
        let url = '/nud-meta.json';
        let indexData = await fetch(url)
            .then((response) => response.json())
            .catch((err) => console.log(err));
        return indexData;
    }

    async fetchTuneIndexData() {
        console.time('index-fetch');

        let url = '/folkfriend-non-user-data.json';

        // Fetch
        let indexData = await fetch(url)
            .then((response) => response.json())
            .catch((err) => console.log(err));

        const downloadedTuneIndex = {
            indexData: indexData
        };

        console.timeEnd('index-fetch');

        return downloadedTuneIndex;
    }

    async setupTuneIndex(cb) {
        // This is the entry point, run every application start, for
        //  loading in the tune index ASAP and also maintaining an up-to-date
        //  offline copy.
        let t0 = performance.now();
        let analyticsData = {
            'newly_installed': false,
            'newly_updated': false
        };
        console.time('tune-index-setup');
        console.time('tune-index-load');

        // This will be null if no tune index has been stored.
        const localTuneIndex = await get('tuneIndex');

        if (typeof localTuneIndex === 'undefined') {
            console.debug('No tune index was cached, requesting download');

            const downloadedTuneIndex = await this.fetchTuneIndexData();

            // Load (so the user can start using the application)
            await this.loadTuneIndex(downloadedTuneIndex);
            console.timeEnd('tune-index-load');

            // Store the version of this newly downloaded tune index
            const tuneIndexMetadata = await this.fetchTuneIndexMetadata();
            await set('tuneIndex', downloadedTuneIndex);
            await set('tuneIndexMetadata', tuneIndexMetadata);

            analyticsData['days_since_update'] = 0;
            analyticsData['tune_index_metadata_version'] = tuneIndexMetadata['v'];
            analyticsData['newly_installed'] = true;
        } else {
            console.debug('Found cached tune index');

            // Load cached copy
            await this.loadTuneIndex(localTuneIndex);
            console.timeEnd('tune-index-load');

            // THEN check the latest version and if we want to upgrade
            const tuneIndexMetadataRemote = await this.fetchTuneIndexMetadata();
            let tuneIndexMetadataLocal = await get('tuneIndexMetadata');

            if (typeof tuneIndexMetadataLocal === 'undefined') {
                // This is a near-impossible state, only reached by people 
                //  selectively deleting from IndexedDB. As browsers do delete
                //   from IndexedDB when under storage pressure it's best to
                //   cover this case and be safe.
                tuneIndexMetadataLocal = {
                    'v': 0
                };
            }

            const remoteVersion = tuneIndexMetadataRemote['v'];
            const localVersion = tuneIndexMetadataLocal['v'];
            const hasUpdate = remoteVersion !== localVersion;
            console.debug(`Tune index remote: ${remoteVersion}, local: ${localVersion}`);

            if (hasUpdate) {
                console.debug('Upgrading tune index');
                const downloadedTuneIndex = await this.fetchTuneIndexData();
                await set('tuneIndex', downloadedTuneIndex);
                await set('tuneIndexMetadata', tuneIndexMetadataRemote);
                analyticsData['days_since_update'] = 0;
                analyticsData['tune_index_metadata_version'] = tuneIndexMetadataRemote['v'];
                analyticsData['newly_updated'] = true;
            } else {
                analyticsData['tune_index_metadata_version'] = tuneIndexMetadataLocal['v'];
            }
        }

        console.timeEnd('tune-index-setup');

        let tEnd = performance.now();
        analyticsData['wall_time'] = tEnd - t0;

        cb(analyticsData);
    }

    async loadTuneIndex(tuneIndex) {
        console.time('tune-index-to-wasm');
        await this.loadedWASM;
        await this.folkfriendWASM.load_index_from_json_obj(tuneIndex.indexData);
        this.setLoadedIndex();
        console.timeEnd('tune-index-to-wasm');
    }

    async setSampleRate(sampleRate) {
        await this.loadedWASM;

        // This can fail by returning false. We never actually check the return
        //  value because it can only fail if passed an invalid sample rate,
        //  and it's trivial to check the sample rate before passing that value
        //  into this worker. It should be impossible for an invalid sample 
        //  rate to make it to the worker, but even if it does the WASM backend
        //  simply ignores the invalid sample rate and stays on the default.
        await this.folkfriendWASM.set_sample_rate(sampleRate);
        this.setLoadedSampleRate();
    }

    async feedEntirePCMSignal(PCMSignal) {
        const frames = Math.floor(PCMSignal.length / ffConfig.SPEC_WINDOW_SIZE);
        if (frames === 0) {
            throw 'PCM signal too short';
        }
        for (let i = 0; i < frames; i++) {
            const PCMWindow = PCMSignal.slice(
                ffConfig.SPEC_WINDOW_SIZE * i,
                ffConfig.SPEC_WINDOW_SIZE * (i + 1)
            );
            await this.feedSinglePCMWindow(PCMWindow);
        }
    }

    async feedSinglePCMWindow(PCMWindow) {
        await this.loadedWASM;
        await this.loadedSampleRate;
        const ptr = await this.folkfriendWASM.alloc_single_pcm_window();
        const arr = await this.folkfriendWASM.get_allocated_pcm_window(ptr);

        arr.set(PCMWindow);

        await this.folkfriendWASM.feed_single_pcm_window(ptr);
        // console.debug("feedSinglePCMWindow: complete");
    }

    async flushPCMBuffer() {
        await this.folkfriendWASM.flush_pcm_buffer();
    }

    async transcribePCMBuffer(cb) {
        try {
            const contour = await this.folkfriendWASM.transcribe_pcm_buffer();
            cb(contour);
        } catch (e) {
            console.error(e);
            console.warn('Aborting transcribePCMBuffer');
            cb(JSON.stringify({
                'error': 'An error ocurred whilst transcribing audio.'
            }));
        }
    }

    async runTranscriptionQuery(query, cb) {
        await this.loadedWASM;
        await this.loadedIndex;
        const response = await this.folkfriendWASM.run_transcription_query(query);
        cb(JSON.parse(response));
    }

    async tuningOffsetCents(cb) {
        await this.loadedWASM;
        cb(this.folkfriendWASM.tuning_offset_cents());
    }

    async transcribePCMBufferSnapshot(cb) {
        try {
            const contour = await this.folkfriendWASM.transcribe_pcm_buffer_snapshot();
            cb(contour);
        } catch (e) {
            console.error(e);
            cb(JSON.stringify({
                'error': 'An error ocurred whilst transcribing audio.'
            }));
        }
    }

    async runNameQuery(query, cb) {
        await this.loadedWASM;
        await this.loadedIndex;
        const response = await this.folkfriendWASM.run_name_query(query);
        cb(JSON.parse(response));
    }

    async contourToAbc(contour, cb) {
        await this.loadedWASM;
        const abc = await this.folkfriendWASM.contour_to_abc(contour);
        cb(abc);
    }

    async settingsFromTuneID(tuneID, cb) {
        await this.loadedWASM;
        await this.loadedIndex;

        const response = await this.folkfriendWASM.settings_from_tune_id(tuneID);
        let settings = JSON.parse(response);

        cb(settings.map(([settingID, setting]) => {
            setting['setting_id'] = settingID;
            return setting;
        }));
    }

    async aliasesFromTuneID(tuneID, cb) {
        await this.loadedWASM;
        await this.loadedIndex;
        const aliases = await this.folkfriendWASM.aliases_from_tune_id(tuneID);
        cb(JSON.parse(aliases));
    }
}

const folkfriendWASMWrapper = new FolkFriendWASMWrapper();
Comlink.expose(folkfriendWASMWrapper);