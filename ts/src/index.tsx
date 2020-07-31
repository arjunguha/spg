import * as React from 'react';
import * as ReactDOM from 'react-dom';

type GalleryImage = {
    'thumbnail_path': string,
    'webview_path': string,
    'original_path': string
}

// Viewing an entire gallery
type GalleryView = {
    'kind': 'gallery',
    'name': string,
    'images': GalleryImage[]
}

// Viewing a single image in a particular gallery
type ImageView = {
    kind: 'image',
    gallery: string,
    image: GalleryImage
}

// Viewing the list of all galleries
type HomeView = {
    kind: 'home',
    galleries: string[]
};

// Initial view, before the list of galleries is loaded
type InitView = {
    kind: 'init'
}

type ErrorView = {
    kind: 'error',
    message: string
}

type View = InitView | HomeView | GalleryView | ImageView | ErrorView;

type State = {
    view: View
};

class Index extends React.Component<{}, State> {
    constructor(props: {}) {
        super(props);
        this.state = {
            view: { 'kind': 'init' }
        };
        window.onpopstate = (event: any) => {
            this.setState(event.state);
        };
        this.handleAsyncError(this.fetchGalleryList());
    }

    // Shows an error message to the user when an asynchronous operation goes
    // wrong. The user can use the Back button to revert to the previous state.
    handleAsyncError(f: Promise<void>) {
        f.then(() => { })
        .catch(exn => {
            window.history.pushState(this.state, '');
            this.setState({ view: { kind: 'error', message: String(exn) } }); 
        });
    }

    async fetchGalleryList(): Promise<void> {
        let resp = await fetch('api/list_galleries');
        let body: string[] = await resp.json();
        window.history.pushState(this.state, '');
        this.setState({ view: { kind: 'home', galleries: body } });
    }

    async fetchGallery(name: string): Promise<void> {
        let resp = await fetch('api/gallery_contents', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(name)
        });
        let body: GalleryImage[] = await resp.json();
        window.history.pushState(this.state, '');
        this.setState({ view: { kind: 'gallery', name: name, images: body } });
    }

    onViewImage(image: GalleryImage, gallery: string) {
        window.history.pushState(this.state, '');
        this.setState({ view: { kind: 'image', gallery: gallery, image: image } });
    }

    makeThumbnail(gallery: string, image: GalleryImage) {
        return <div className="thumbnail">
            <img src={`photos/${image.thumbnail_path}`}
                onClick={() => this.onViewImage(image, gallery)}></img>
            </div>;
    }

    renderImage(image: ImageView) {
        return (<div>
            <h1>{image.gallery}</h1>
                <div>
                    <a href="#" onClick={() => this.handleAsyncError(this.fetchGallery(image.gallery))}>Return to gallery</a>
                </div>
                <div>{image.image.original_path}</div>
                <img src={`photos/${image.image.webview_path}`}></img>
                <div>
                    <a href="#" onClick={() => this.handleAsyncError(this.fetchGallery(image.gallery))}>Return to gallery</a>
                </div>
            </div>);
    }

    renderGallery(gallery: GalleryView) {
        let thumbnails = gallery.images.map(image => this.makeThumbnail(gallery.name, image));
        return (<div>
            <h1>{gallery.name}</h1>
            <a href="#" onClick={() => this.handleAsyncError(this.fetchGalleryList())}>Home</a>
            <br/>
            {thumbnails}
            </div>
            );
    }

    render() {
        if (this.state.view.kind === 'init') {
            return (<div>Loading ...</div>);
        }
        else if (this.state.view.kind === 'error') {
            return (<div>
                <h1>Something Went Wrong</h1>
                <div>Press back to return where you were.</div>
                <div>{this.state.view.message}</div>
                </div>);
        }
        else if (this.state.view.kind === 'home') {
            let links = this.state.view.galleries.map(name => 
                <li> <a href="#" onClick={() => this.handleAsyncError(this.fetchGallery(name))}>{name}</a></li>);
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
            throw new Error('Unhandled state in render()');
        }

    }   
    
}

ReactDOM.render(<Index/>, document.querySelector('#root'));