<script lang="ts">
    import CodeMirror from "svelte-codemirror-editor";
    import { defaultKeymap } from "@codemirror/commands";
    import { keymap } from "@codemirror/view";
	import { onDestroy } from "svelte";
	import { browser } from "$app/environment";
    
    let value = "";
    let name = "";
    let lastValue = "";

    if (browser) {

        const urlParams = new URLSearchParams(window.location.search);
        const host = urlParams.get('host') ?? "http://127.0.0.1";

        async function postCode() {
            if (lastValue == value) {
                return;
            }
            lastValue = value;
            let url;
            if (name.length > 0) {
                url = `${host}/code/${name}/`;
            } else {
                url = `${host}/code/`;
            }
            fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "text/plain",
                },
                body: value,
                credentials: 'include'
            });
        }

        const interval = setInterval(postCode, 2000);

        onDestroy(() => {
            clearInterval(interval);
        });
    }

</script>

<label for=name>Name:</label>
<input id=name bind:value={name} on:change={() => lastValue = ""}>
<hr>
<CodeMirror bind:value basic={false} extensions={[keymap.of([
    ...defaultKeymap,
  ])
]}/>