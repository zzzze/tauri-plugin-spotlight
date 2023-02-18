import { useState } from 'react'
import { invoke } from '@tauri-apps/api/tauri'
import './App.css'

function App () {
  const [greetMsg, setGreetMsg] = useState('')
  const [name, setName] = useState('')

  async function greet () {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke('greet', { name }))
  }

  return (
    <div className="container">
      <div className="row">
        <form
          onSubmit={(e) => {
            e.preventDefault()
            void greet()
          }}
        >
          <input
            id="greet-input"
            onChange={(e) => { setName(e.currentTarget.value) }}
            placeholder="Enter a name..."
            autoFocus
          />
          <button type="submit">Greet</button>
        </form>
      </div>
      <p>{greetMsg}</p>
    </div>
  )
}

export default App
