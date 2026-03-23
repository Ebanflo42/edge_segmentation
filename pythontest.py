import numpy as np
from PIL import Image
from scipy.ndimage import sobel
from sklearn.mixture import GaussianMixture

with Image.open("funhouse.jpg") as im:
    arr = np.array(im)

print(arr.shape)
arr = arr.sum(axis=-1)
sobel_magnitude = np.sqrt(sobel(arr, axis=0)**2 + sobel(arr, axis=1)**2)
sobel_thresh = np.where(sobel_magnitude > 0.3)
gmm = GaussianMixture(n_components=1024, covariance_type=)