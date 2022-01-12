export interface Device {
  vendorId: number,
  productId: number,
  deviceName: string,
  manufacturer: string,
  serialNumber: string,
  deviceAddress: number
}

export function find(params?: { vid?: number, pid?: number}): Device[]