export const source_pool = [
  "https://i.imgur.com/892vhef.jpeg",
  "https://avatars.mds.yandex.net/i?id=afb182659773be12c48e7a49d7e8212c_l-5858046-images-thumbs&n=13",
  "https://avatars.mds.yandex.net/i?id=3ef58cad5f77fcebe674582d17765372_l-4032453-images-thumbs&n=13",
  "https://avatars.mds.yandex.net/i?id=cc49c0be94d8640bd74a7a7a4ba48dfd_l-2396749-images-thumbs&n=13",
];

export type PreviewData = {
  title: string;
  donation_buttons: {
    name: string;
    description: string;
    amount: number;
    source_id: number;
    invoice_url: string;
  }[];
};

export const preview_default: PreviewData = {
  title: "Header Text",
  donation_buttons: [
    {
      name: "Donate 1",
      description: "Description 1",
      amount: 100,
      source_id: 0,
      invoice_url: "https://t.me/$clgGxe0mwEq9CwAAvJUBv--iitU",
    },
    {
      name: "Donate 2",
      description: "Description 2",
      amount: 200,
      source_id: 1,
      invoice_url: "https://t.me/$pHcweO0mwEq-CwAATVxs8DbroT0",
    },
    {
      name: "Donate 3",
      description: "Description 3",
      amount: 300,
      source_id: 2,
      invoice_url: "https://t.me/$HYySbu0mwErACwAA5Pxvbym2xzw",
    },
    {
      name: "Donate 4",
      description: "Description 4",
      amount: 400,
      source_id: 3,
      invoice_url: "https://t.me/$RJw1Ye0mwErBCwAA2omxPGrT2II",
    },
  ],
};
