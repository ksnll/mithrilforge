/* @refresh reload */
import { render } from 'solid-js/web';

import './index.css';
import App from './App';
import { Auth0 } from '@rturnq/solid-auth0';

export const API_URL = { 'localhost:3000': 'http://localhost:5558/api', 'mithrilforge.ksnll.com': 'https://mithrilforge.ksnll.com/api' }[window.location.host]!;
const AUTH0_CONFIG = {
  'localhost:3000': {
    audience: 'http://localhost:5558',
    clientId: '5C1DA8ZU0rMtyha0DEDTRxs1phEajjry',
    domain: 'ksnll.eu.auth0.com'
  },
  'mithrilforge.ksnll.com': {

    audience: 'https://mithrilforge.ksnll.com/',
    clientId: '5C1DA8ZU0rMtyha0DEDTRxs1phEajjry',
    domain: 'ksnll.eu.auth0.com'

  }
}[window.location.host]!;

const root = document.getElementById('root');

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    'Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?',
  );
}

render(() => (<Auth0
  domain={AUTH0_CONFIG.domain}
  clientId={AUTH0_CONFIG.clientId}
  audience={AUTH0_CONFIG.audience}
  logoutRedirectUri={`${window.location.origin}/logout`}
  loginRedirectUri={`${window.location.origin}/`}
><App /></Auth0>), root!);
