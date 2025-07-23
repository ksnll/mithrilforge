import { createEffect, createResource, createSignal, Show } from 'solid-js';
import { Send } from './icons/Send';
import { useAuth0 } from '@rturnq/solid-auth0';
import { Websites } from './Websites';
import { API_URL } from '.';
const ENTER_KEY = 'Enter';


const App = () => {
  const [name, setName] = createSignal("");
  const auth = useAuth0();
  const handleSubmit = async () => {
    try {
      const token = await auth?.getToken();
      const res = await fetch(`${API_URL}/website`, {
        method: "POST",
        headers: { "Content-Type": "application/json", "Authorization": `Bearer ${token}` },
        body: JSON.stringify({ source_address: name() }),
      });

      if (!res.ok) throw new Error(`Server replied ${res.status}`);
      const data = await res.json();
      console.log("Server response:", data);
    } catch (err) {
      console.error("POST failed:", err);
    }
  };
  createEffect(() => {
    if (auth?.isInitialized() && !auth.isAuthenticated()) {
      auth.loginWithRedirect();
    }
  });
  return (<Show when={auth?.isInitialized()}>
    <div id="main-container">
      <h1>MithrilForge</h1>
      <p>Reforging shitty websites, one at a time.</p>
      <div class="mt-10">
        <div class="input-container"><input type="text" placeholder="Enter a website URL..." value={name()} onInput={e => setName(e.currentTarget.value)} onKeyDown={e => {
          if (e.key === ENTER_KEY) { e.preventDefault(); handleSubmit(); }
        }} />
          <button type="submit" onClick={handleSubmit}>
            <Send />
          </button>
        </div>
        <Websites />
      </div>
    </div></Show>
  );
};

export default App;
