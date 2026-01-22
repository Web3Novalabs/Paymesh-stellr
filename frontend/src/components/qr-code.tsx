export default function qrCode(): string {
  return `
    <svg width="128" height="128" viewBox="0 0 128 128" fill="none" xmlns="http://www.w3.org/2000/svg">
      <rect width="128" height="128" fill="white"/>
      <path d="M16 16H48V48H16V16Z" fill="black"/>
      <path d="M32 32H48V48H32V32Z" fill="white"/>
      <path d="M16 64H48V96H16V64Z" fill="black"/>
      <path d="M32 80H48V96H32V80Z" fill="white"/>
      <path d="M80 16H112V48H80V16Z" fill="black"/>
      <path d="M96 32H112V48H96V32Z" fill="white"/>
      <path d="M64 64H96V96H64V64Z" fill="black"/>
      <path d="M80 80H96V96H80V80Z" fill="white"/>
      <path d="M16 112H48V128H16V112Z" fill="black"/>
      <path d="M32 112H48V128H32V112Z" fill="white"/>
      <path d="M80 112H112V128H80V112Z" fill="black"/>
      <path d="M96 112H112V128H96V112Z" fill="white"/>
      <path d="M64 16H80V32H64V16Z" fill="black"/>
      <path d="M16 80H32V96H16V80Z" fill="black"/>
      <path d="M112 16H128V32H112V16Z" fill="black"/>
      <path d="M16 48H32V64H16V48Z" fill="black"/>
      <path d="M112 64H128V80H112V64Z" fill="black"/>
    </svg>
  `;
}