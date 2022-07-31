"use strict"

import { get } from "./fetch"

interface VideoDecoder {
  dstWidth: number
  dstHeight: number
  canvas: HTMLCanvasElement
  context: CanvasRenderingContext2D | null
  video: HTMLVideoElement
}

class VideoDecoder {
  constructor(dstWidth: number, dstHeight: number) {
    this.dstWidth = dstWidth
    this.dstHeight = dstHeight

    this.canvas = document.createElement("canvas")
    this.canvas.width = dstWidth
    this.canvas.height = dstHeight
    this.context = this.canvas.getContext("2d")

    if (this.context) {
      this.context.imageSmoothingEnabled = false
    }

    this.video = document.createElement("video")
    this.video.muted = true
    this.video.crossOrigin = "anonymous" // TODO: maybe remove this? ask jokler
  }

  async decode(url: string): Promise<string> {
    this.video.src = url

    const getFrame = new Promise((resolve, reject) => {
      // This gets a video frame
      this.video
        .play()
        .catch((e) => reject(e))
        .then(() => this.video.pause())

      this.video.onpause = () => {
        let newWidth = this.dstWidth
        let newHeight = this.dstHeight

        if (this.video.videoWidth > this.video.videoHeight) {
          newHeight = (this.dstWidth / this.video.videoWidth) * this.video.videoHeight
        } else {
          newWidth = (this.dstHeight / this.video.videoHeight) * this.video.videoWidth
        }

        this.canvas.width = newWidth
        this.canvas.height = newHeight

        if (this.context) {
          this.context.drawImage(this.video, 0, 0, this.video.videoWidth, this.video.videoHeight, 0, 0, newWidth, newHeight) // prettier-ignore
        }

        resolve(this.canvas)
      }
    })

    return await getFrame
      .then((frame: any) => {
        return frame.toDataURL()
      })
      .catch((e) => {
        console.log(e)
        return ""
      })
  }
}

const decoder = new VideoDecoder(180, 120)
let interval: NodeJS.Timeout

export async function startStreamsUpdate() {
  const streams = await get("/api/streams")

  console.log(streams)
}

export async function stopStreamsUpdate() {}
