<script lang="ts">
    import CodeMirror from "svelte-codemirror-editor";
    import { defaultKeymap } from "@codemirror/commands";
    import { keymap } from "@codemirror/view";
	import { onDestroy } from "svelte";
	import { browser } from "$app/environment";

    function generateRandomString(length: number): string {
        const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
        let result = '';
        const charactersLength = characters.length;
        for (let i = 0; i < length; i++) {
            result += characters.charAt(Math.floor(Math.random() * charactersLength));
        }
        return result;
    }
    
    let value = "";
    let name = "";
    let lastValue = "";

    if (browser) {
        let id = generateRandomString(16);
        const urlParams = new URLSearchParams(window.location.search);
        const host = urlParams.get('host') ?? "http://127.0.0.1";

        async function postCode() {
            if (lastValue == value) {
                return;
            }
            lastValue = value;
            let url;
            if (name.length > 0) {
                url = `${host}/code/${id}/${name}/`;
            } else {
                url = `${host}/code/${id}/`;
            }
            fetch(url, {
                method: "POST",
                headers: {
                    "Content-Type": "text/plain",
                },
                body: value,
                mode: 'no-cors',
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