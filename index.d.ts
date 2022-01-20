import { Emitter, Events } from 'mitt'

interface Device {
  vendorId: number
  productId: number
  deviceName: string
  manufacturer: string
  serialNumber: string
  deviceAddress: number
}

type Action = 'add' | 'change' | 'remove'
type VidPid = `:${number}` | `:${number}:${number}`
type Add = `${Action}${VidPid}`

type Events = {
  add: Device
  change: Device
  remove: Device
  [key: Add]: Device
}

interface DetectEmitter extends Emitter<Events> {
  find(params?: { vid?: number; pid?: number }): Promise<Device[]>
  startMonitoring(): void
  stopMonitoring(): void
}

declare const detect: DetectEmitter

export = detect
