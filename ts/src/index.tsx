import * as React from 'react';
import * as ReactDOM from 'react-dom';

type GalleryImage = {
    'thumbnail_path': string,
    'webview_path': string
}

type GalleryView = {
    'kind': 'gallery',
    'name': string,
    'images': GalleryImage[]
}

type ImageView = {
    kind: 'image',
    gallery: string,
    image: GalleryImage
}

type View = { 'kind': 'home' } | GalleryView | ImageView;

type State = {
    galleries: string[],
    view: View
};

class Index extends React.Component<{}, State> {
    constructor(props: {}) {
        super(props);
        this.state = {
            galleries: [],
            view: { 'kind': 'home' }
        };
        this.fetchListGalleries(); 
    }

    async fetchListGalleries(): Promise<void> {
        let resp = await fetch('/api/list_galleries');
        let body: string[] = await resp.json();
        this.setState({ galleries: body });
    }

    async fetchGalleryAsync(name: string): Promise<void> {
        let resp = await fetch('/api/gallery_contents', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(name)
        });
        let body: GalleryImage[] = await resp.json();
        this.setState({ view: { kind: 'gallery', name: name, images: body } });
    }

    fetchGallery(name: string) {
        this.fetchGalleryAsync(name);
    }

    makeThumbnail(gallery: string, image: GalleryImage) {
        let viewOnClick: ImageView = {
            kind: 'image',
            gallery: gallery,
            image: image
        };
        return <img src={image.thumbnail_path} 
            onClick={() => this.setState({ view: viewOnClick })}></img>;
    }

    renderImage(image: ImageView) {
        return (<div>
            <h1>{image.gallery}</h1>
            <a href="#" onClick={() => this.fetchGallery(image.gallery)}>Back</a>
            <img src={image.image.webview_path}></img>
            </div>);
    }

    renderGallery(gallery: GalleryView) {
        let thumbnails = gallery.images.map(image => this.makeThumbnail(gallery.name, image));
        return (<div>
            <h1>{gallery.name}</h1>
            <a href="#" onClick={() => this.setState({ view: { kind: 'home' } })}>Home</a>
            <br/>
            {thumbnails}
            </div>
            );
    }

    render() {
        if (this.state.view.kind === 'home') {
            let links = this.state.galleries.map(name => 
                <li> <div onClick={() => this.fetchGallery(name)}>{name}</div></li>);
            return (<div>
                <h1>Home</h1>
                <ul>{links}</ul>
                </div>);
        }
        else if (this.state.view.kind === 'gallery') {
            return this.renderGallery(this.state.view);
        }
        else if (this.state.view.kind === 'image') {
            return this.renderImage(this.state.view);
        }
        else {
            throw new Error('');
        }

    }   
    
}

ReactDOM.render(<Index/>, document.querySelector('#root'));