import { createEffect, createResource, For, onCleanup } from "solid-js";
import { API_URL } from ".";
import { useAuth0 } from "@rturnq/solid-auth0";
import { createStore } from "solid-js/store";


type Website = {
  id: number,
  source_address: string,
  contact_name?: string;
  contact_email?: string;
  gpt_prompt?: string;
  lovable_link?: string;
};
type GetWebsites = { "websites": Website[] };

export const Websites = () => {
  const [state, setState] = createStore([] as Website[]);
  const auth = useAuth0();

  createEffect(async () => {
    const token = await auth?.getToken();
    if (!token)
      return;

    const ws = new WebSocket(`${API_URL}/events?token=${encodeURIComponent(token)}`);

    ws.onmessage = ({ data }) => {
      const message = JSON.parse(data);
      switch (message.type) {
        case "WebsiteAdded": {
          const { id, source_address } = message;
          setState(websites => [{ id, source_address }, ...websites]);
          break;
        }
        case "FetchedContact": {
          const {
            website_id,
            contact: { contact_email, contact_name },
          } = message;

          setState(
            w => w.id === website_id,
            { contact_name, contact_email },
          );
          break;
        }
      }
    };

    onCleanup(() => ws.close());
  });


  createResource(async () => {
    const token = await auth?.getToken();
    const response = await fetch(`${API_URL}/websites`, {
      headers: { "Content-Type": "application/json", "Authorization": `Bearer ${token}` },
    });
    const websites = (await response.json()) as GetWebsites;
    setState(websites.websites)
  });
  return <For each={state}>{(website) => <div class="card"><h2>{website.source_address}</h2>
    <div class="content">
      <div class="website">
        <p><strong>Contact email: </strong></p>
        {website.contact_email ? <span class="ml-3">{website.contact_email}</span> : <div class="spinner" role="status" aria-label="Loading"></div>}
      </div>

      <div class="website">
        <p><strong>Contact name: </strong></p>
        {website.contact_name ? <span class="ml-3">{website.contact_name}</span> : <div class="spinner" role="status" aria-label="Loading"></div>}
      </div>

      <div class="website">
        <p><strong>GPT prompt:</strong></p>
        <div class="spinner" role="status" aria-label="Loading"></div></div>


      <div class="website">
        <p><strong>Lovable link:</strong></p>
        <div class="spinner" role="status" aria-label="Loading"></div></div>
    </div>
  </div>}</For>;
}
