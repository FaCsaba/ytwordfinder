import { useEffect, useState } from 'react'
import './App.css'

interface VideoTime {
  content: string,
  link: string
}

function App() {
  const [videos, setVideos] = useState<VideoTime[]>([]);
  const [currentVideo, setCurrentVideo] = useState<number>(0);
  const [searchWord, setSearchWord] = useState<string | undefined>();
  const [err, setErr] = useState<boolean>(false);

  useEffect(() => {
    if (!searchWord) {return}
    fetch("/api/getVideoLinks/" + searchWord)
    .then(r => r.json())
    .then((videoTimes: VideoTime[]) => {
      setVideos(videoTimes);
      setCurrentVideo(0);
    })
    .catch(() => setErr(true));
  }, [searchWord])

  function next() {
    if (!videos[currentVideo + 1]) {setCurrentVideo(0)};
    setCurrentVideo((v) => v+=1);
  }

  return (
    <> {!err &&
      <>
        <div style={{display: 'flex', flexDirection: 'row', justifyContent: 'space-between', height: '3rem'}}>
                <input type="text" style={{fontSize: '2rem', padding: '10px'}} onChange={(e) => {setSearchWord(e.target.value)}} />
                {videos.length > 0 && searchWord && <h3 style={{margin: 'auto 0'}}>{currentVideo + 1} of {videos.length} videos</h3>}
              </div>
              <div style={{width: '1000px', height: '600px', display: 'flex'}}>
              {videos[currentVideo] && 
                <iframe 
                id='player'
                width="1000"
                height="600"
                src={videos[currentVideo].link}
                title="YouTube video player"
                frameBorder="0"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen /> ||
                <h1 style={{margin: 'auto'}}>Try searching something</h1>
              }
              </div>
              {videos[currentVideo] &&
                <h2>{videos[currentVideo].content}</h2>
              }
              <button onClick={next}>Next</button>
        </>
      }
      {err && 
        <h1>An error occurred</h1>
      }
    </>
  )
}

export default App
