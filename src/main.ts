/**
 * Punto de entrada del frontend (composition root).
 * En F1 aquí se crearán los modelos y controladores y se inyectarán en las vistas.
 * @packageDocumentation
 */
import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
